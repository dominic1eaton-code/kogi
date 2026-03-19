import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubVotingComponent } from './hub-voting.component';

describe('HubVotingComponent', () => {
  let component: HubVotingComponent;
  let fixture: ComponentFixture<HubVotingComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubVotingComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubVotingComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
