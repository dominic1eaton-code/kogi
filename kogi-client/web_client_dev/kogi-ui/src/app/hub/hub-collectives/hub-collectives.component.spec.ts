import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubCollectivesComponent } from './hub-collectives.component';

describe('HubCollectivesComponent', () => {
  let component: HubCollectivesComponent;
  let fixture: ComponentFixture<HubCollectivesComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubCollectivesComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubCollectivesComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
