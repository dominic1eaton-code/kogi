import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubTeamsComponent } from './hub-teams.component';

describe('HubTeamsComponent', () => {
  let component: HubTeamsComponent;
  let fixture: ComponentFixture<HubTeamsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubTeamsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubTeamsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
